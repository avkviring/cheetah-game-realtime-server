using System;
using System.Net;
using System.Threading;
using System.Threading.Tasks;
using Docker.DotNet;
using Docker.DotNet.Models;
using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    /// <summary>
    ///     Читает логи из docker контейнера и отображает их в консоле Unity
    /// </summary>
    public class DockerLogWatcher : IDisposable
    {
        /// <summary>
        ///     Время ожидания перед запуском нового цикла чтения логов
        /// </summary>
        public static TimeSpan FetchTime = TimeSpan.FromSeconds(2);

        private readonly bool showInfoLogs = false;
        private static volatile bool running = true;
        private readonly CancellationTokenSource cancellationTokenSource = new CancellationTokenSource();
        private DockerClient dockerClient;

        public DockerLogWatcher(DockerClient dockerClient, bool showInfoLogs)
        {
            this.showInfoLogs = showInfoLogs;
            this.dockerClient = dockerClient;
        }

        public void Dispose()
        {
            running = false;
            cancellationTokenSource.Cancel();
            cancellationTokenSource.Dispose();
        }

        public void WatchLogs(string id, ServerApplication serverApplication)
        {
            var taskScheduler = TaskScheduler.FromCurrentSynchronizationContext();
            Task.Factory.StartNew(async () => { await ReadLogs(id, serverApplication); }, cancellationTokenSource.Token,
                TaskCreationOptions.None, taskScheduler);
        }

        /// <summary>
        /// Необходим для уменьшения количества соединений к docker, так как количество Pipe в Windows ограничено.
        /// </summary>
        public static bool inReading = false;

        private async Task ReadLogs(string id, ServerApplication serverApplication)
        {
            long since = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            try
            {
                while (running)
                {
                    while (inReading)
                    {
                        await Task.Yield();
                    }

                    inReading = true;
                    var logs = await dockerClient.Containers.GetContainerLogsAsync(
                        id,
                        false,
                        new ContainerLogsParameters
                        {
                            ShowStderr = true,
                            ShowStdout = true,
                            Follow = false,
                            Since = since.ToString()
                        },
                        cancellationTokenSource.Token);
                    since = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
                    var (stdout, stderr) = await logs.ReadOutputToEndAsync(cancellationTokenSource.Token);
                    ProcessLog(serverApplication, stdout);
                    ProcessLog(serverApplication, stderr);
                    inReading = false;
                    await Task.Delay(FetchTime);
                }
            }
            catch (DockerApiException e)
            {
                // 404, 409 - контейнер не существует, нормальная ситуация при останове сервисов
                if (e.StatusCode != HttpStatusCode.NotFound && e.StatusCode != HttpStatusCode.Conflict) throw;
            }
            catch (TaskCanceledException)
            {
                // нормальная ситуация при перезапуске домена
                throw;
            }
            catch (Exception e)
            {
                Debug.LogWarning(e);
                throw;
            }
            finally
            {
                inReading = false;
            }
        }

        private void ProcessLog(ServerApplication serverApplication, string log)
        {
            if (log.Trim().Length == 0)
            {
                return;
            }

            var logItem = serverApplication.ConvertToLogItem(log);
            if (logItem == null) return;

            var logValue = logItem.Value;
            var message = "[" + serverApplication.ContainerName + "]: " + logValue.Log;
            switch (logValue.ItemType)
            {
                case ServerApplication.LogItemType.Info:
                    if (showInfoLogs)
                    {
                        Debug.Log(message);
                    }

                    break;
                case ServerApplication.LogItemType.Error:
                    Debug.LogError(message);
                    break;
                case ServerApplication.LogItemType.Message:
                    Debug.Log(message);
                    break;
            }
        }
    }
}