using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Docker.DotNet;
using Docker.DotNet.Models;
using UnityEditor;
using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    /// <summary>
    /// Запускаем серверные приложения в Docker
    /// </summary>
    public class PlatformInDockerRunner : IDisposable
    {
        public delegate void ChangeStatus(Status status);

        private const string unityProjectId = "cheetah";

        private Status _status;

        private readonly DockerClient docker;

        private readonly DockerLogWatcher logWatcher;


        public PlatformInDockerRunner()
        {
            Status = Status.Unknown;

            DockerClientConfiguration dockerClientConfiguration;
            try
            {
                var dockerUrl = Environment.GetEnvironmentVariable("DOCKER_HOST");
                dockerClientConfiguration = new DockerClientConfiguration(new Uri(dockerUrl));
            }
            catch (ArgumentException)
            {
                dockerClientConfiguration = new DockerClientConfiguration();
            }
            docker = dockerClientConfiguration.CreateClient();

            logWatcher = new DockerLogWatcher(docker);
            AssemblyReloadEvents.beforeAssemblyReload += Dispose;

            var taskScheduler = TaskScheduler.FromCurrentSynchronizationContext();
            Task.Factory.StartNew(
                async () => { await ResolveState(); },
                CancellationToken.None,
                TaskCreationOptions.None,
                taskScheduler);
        }

        public Status Status
        {
            get => _status;
            private set
            {
                _status = value;
                OnStatusChange?.Invoke(_status);
            }
        }


        public void Dispose()
        {
            logWatcher?.Dispose();
        }

        public event ChangeStatus OnStatusChange;


        /// <summary>
        ///     Определение состояния после перезапуска Unity
        ///     - проверяем все ли контейнеры запущены
        /// </summary>
        /// <returns></returns>
        public async Task ResolveState()
        {
            if (Status != Status.Unknown) return;

            try
            {
                var serverApplications = Registry.GetApplications().ToDictionary(c => c.Name);
                var existContainers = await docker.Containers.ListContainersAsync(new ContainersListParameters
                {
                    Filters = new Dictionary<string, IDictionary<string, bool>>
                    {
                        ["label"] = new Dictionary<string, bool>
                        {
                            [DockerContainerBuilder.DockerProjectLabel + "=" + unityProjectId] = true
                        }
                    },
                    All = true
                });


                foreach (var existContainer in existContainers)
                {
                    var label = existContainer.Labels[DockerContainerBuilder.DockerNameLabel];
                    if (label != null)
                        if (serverApplications.TryGetValue(label, out var serverApplication))
                            if (serverApplication.DockerImage.Ref == existContainer.Image)
                            {
                                serverApplications.Remove(label);
                                logWatcher.WatchLogs(existContainer.ID, serverApplication);
                            }
                }

                Status = serverApplications.Any() ? Status.Stopped : Status.Started;
            }
            catch (HttpRequestException e)
            {
                Debug.LogException(e);
                Status = Status.Fail;
                throw;
            }
            catch (Exception e)
            {
                Debug.LogError(e);
                Status = Status.Fail;
            }
        }

        public async Task Restart(IDockerProgressListener progressListener)
        {
            Status = Status.Starting;
            try
            {
                progressListener.SetProgressTitle("remove previous");
                progressListener.SetProgress(0);

                // удаляем все, что было запущено
                await Remove(docker, progressListener);

                progressListener.SetProgressTitle("create network");
                progressListener.SetProgress(0);
                var network = await CreateNetwork(unityProjectId, docker);

                var progress = 0;
                var serverApplications = Registry.GetApplications();
                await LaunchYDB(progressListener, serverApplications, progress, network);
                var deltaProgress = 90 / serverApplications.Count; // 10 процентов - на запуск nginx
                var done = false;
                var launched = new HashSet<string>();
                while (!done)
                {
                    done = true;

                    progressListener.SetProgressTitle("starting services, remaining " + serverApplications.Count);
                    progressListener.SetProgress(progress);
                    foreach (var serverApplication in serverApplications.Where(application =>
                        {
                            var dependencies = new HashSet<string>(application.Dependencies);
                            dependencies.RemoveWhere(p => launched.Contains(p));
                            return dependencies.Count == 0;
                        }
                    ).ToList())
                    {
                        await Launch(serverApplication, network.ID, progressListener);
                        serverApplications.Remove(serverApplication);
                        launched.Add(serverApplication.Name);
                        done = false;
                        progress += deltaProgress;
                    }


                    if (done && serverApplications.Count > 0)
                    {
                        Debug.LogError("Cannot resolve dependencies for containers");
                        await Remove(docker, progressListener);
                        Status = Status.Fail;
                        return;
                    }
                }

                Status = Status.Started;
            }
            catch (HttpRequestException e)
            {
                Status = Status.Disconnected;
                throw new DockerConnectException(e);
            }
            catch (SocketException e)
            {
                Status = Status.Disconnected;
                throw new DockerConnectException(e);
            }
            catch (Exception)
            {
                Status = Status.Fail;
                await Task.Delay(DockerLogWatcher.FetchTime.Add(DockerLogWatcher.FetchTime));
                throw;
            }
            finally
            {
                if (Status == Status.Fail)
                {
                    // ожидаем получание логов
                    await Task.Delay(DockerLogWatcher.FetchTime.Add(DockerLogWatcher.FetchTime));
                    await Remove(docker, progressListener);
                    Status = Status.Fail;
                }
            }

            await Task.Delay(TimeSpan.FromSeconds(10));
        }

        private async Task LaunchYDB(IDockerProgressListener progressListener, List<ServerApplication> serverApplications, int progress,
            NetworksCreateResponse network)
        {
            var applicationWithPostgresql = serverApplications.FindAll(app => app.YDBEnabled);
            if (applicationWithPostgresql.Count > 0)
            {
                progressListener.SetProgressTitle("starting yandex database");
                progressListener.SetProgress(progress);
                await Launch(new YandexDBApplication(), network.ID, progressListener);
            }
        }


        private async Task<string> Launch(ServerApplication serverApplication, string networkId, IDockerProgressListener progressListener)
        {
            await ImagePull(serverApplication.DockerImage, progressListener, serverApplication.Name);
            var dockerContainerBuilder = new DockerContainerBuilder(serverApplication.Name, serverApplication.DockerImage);
            if (serverApplication.YDBEnabled) serverApplication.ConfigureYDBEnv(dockerContainerBuilder);
            serverApplication.ConfigureDockerContainerBuilder(dockerContainerBuilder);

            var createContainerResponse =
                await docker.Containers.CreateContainerAsync(dockerContainerBuilder.BuildDockerConfig(networkId, unityProjectId));
            logWatcher.WatchLogs(createContainerResponse.ID, serverApplication);
            var containerStarted = await docker.Containers.StartContainerAsync(createContainerResponse.ID, new ContainerStartParameters());
            if (!containerStarted) throw new Exception("Container " + serverApplication.Name + " starting fail");
            await WaitingForLaunch.Wait(docker, createContainerResponse.ID, serverApplication.Name);
            return createContainerResponse.ID;
        }

        private async Task ImagePull(DockerImage dockerImage, IDockerProgressListener progressListener, string title)
        {
            var listImagesParameters = new ImagesListParameters
            {
                Filters = new Dictionary<string, IDictionary<string, bool>>
                {
                    ["reference"] = new Dictionary<string, bool>
                    {
                        [dockerImage.Ref] = true
                    }
                }
            };
            if ((await docker.Images.ListImagesAsync(listImagesParameters)).Count > 0)
            {
                return;
            }

            await docker.Images.CreateImageAsync(new ImagesCreateParameters
            {
                FromImage = dockerImage.Ref
            },
                null,
                new ImageCreateProgress(progressListener, title));
        }


        private static async Task<NetworksCreateResponse> CreateNetwork(string instanceId, DockerClient docker)
        {
            var networkConfig = new NetworksCreateParameters
            {
                Name = "cheetah_platform_network_" + instanceId,
                Labels = new Dictionary<string, string> { [DockerContainerBuilder.DockerProjectLabel] = instanceId }
            };
            return await docker.Networks.CreateNetworkAsync(networkConfig);
        }

        private async Task Remove(DockerClient docker, IDockerProgressListener progressListener)
        {
            // удаляем контейнеры
            var filters = new Dictionary<string, IDictionary<string, bool>>
            {
                ["label"] = new Dictionary<string, bool>
                {
                    [DockerContainerBuilder.DockerProjectLabel + "=" + unityProjectId] = true
                }
            };
            var containers = await docker.Containers.ListContainersAsync(new ContainersListParameters
            {
                All = true,
                Filters = filters
            });
            if (containers.Count > 0)
            {
                var progress = 0;
                var progressDelta = 80 / containers.Count;
                foreach (var container in containers)
                {
                    if (container.State == "Up")
                    {
                        progressListener.SetProgressTitle("stop " + container.Names.First());
                        progressListener.SetProgress(progress);
                        await docker.Containers.StopContainerAsync(container.ID, new ContainerStopParameters
                        {
                            WaitBeforeKillSeconds = 1
                        });
                    }

                    progressListener.SetProgressTitle("delete " + container.Names.First());
                    progressListener.SetProgress(progress);
                    await docker.Containers.RemoveContainerAsync(container.ID, new ContainerRemoveParameters
                    {
                        Force = true,
                        RemoveLinks = false,
                        RemoveVolumes = false
                    });
                    progress += progressDelta;
                }
            }

            progressListener.SetProgressTitle("stop network");
            progressListener.SetProgress(80);

            var networks = await docker.Networks.ListNetworksAsync(new NetworksListParameters
            {
                Filters = filters
            });
            foreach (var network in networks) await docker.Networks.DeleteNetworkAsync(network.ID);

            progressListener.SetProgressTitle("stopped network");
            progressListener.SetProgress(100);
        }

        public async Task Stop(IDockerProgressListener progressListener)
        {
            try
            {
                Status = Status.Stopping;
                progressListener.SetProgressTitle("stop");
                progressListener.SetProgress(0);
                await Remove(docker, progressListener);
                progressListener.SetProgressTitle("stopped");
                progressListener.SetProgress(100);
            }
            catch (HttpRequestException e)
            {
                throw new DockerConnectException(e);
            }
            finally
            {
                Status = Status.Stopped;
            }
        }
    }

    public enum Status
    {
        Unknown,
        Starting,
        Started,
        Stopping,
        Stopped,
        Fail,
        Disconnected
    }
}