using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Docker.DotNet.Models;
using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    /// <summary>
    ///     Builder для создания конфигурации контейнера для Docker Remote Api
    /// </summary>
    public sealed class DockerContainerBuilder
    {
        public const string DockerProjectLabel = "games.cheetah.docker.unityProject";
        public const string DockerNameLabel = "games.cheetah.docker.name";

        private readonly List<string> commands = new List<string>();


        private readonly IList<string> Env = new List<string>();
        private HealthConfig healthConfig;
        private readonly IList<PortMapping> portBindingsConfig = new List<PortMapping>();
        private readonly IDictionary<string, string> VolumeMappings = new Dictionary<string, string>();


        public DockerContainerBuilder(string name, DockerImage image)
        {
            Name = name;
            Image = image;
        }

        private string Name { get; }
        private DockerImage Image { get; }


        public CreateContainerParameters BuildDockerConfig(string networkId, string unityProjectId)
        {
            var portBindings = new Dictionary<string, IList<PortBinding>>();
            foreach (var portBinding in portBindingsConfig)
            {
                var bindings = new List<PortBinding>();
                portBindings[portBinding.ContainerPort + "/" + Enum.GetName(typeof(Protocol), portBinding.Protocol)?.ToLower()] = bindings;

                bindings.Add(new PortBinding
                {
                    HostIP = portBinding.HostAddress,
                    HostPort = portBinding.HostPort.ToString()
                });
            }

            var binds = VolumeMappings.Select(mapping => mapping.Key + ":" + mapping.Value).ToList();

            return new CreateContainerParameters
            {
                Name = Name,
                Image = Image.Ref,
                Labels = new Dictionary<string, string>
                {
                    [DockerProjectLabel] = unityProjectId,
                    [DockerNameLabel] = Name
                },
                NetworkingConfig = new NetworkingConfig
                {
                    EndpointsConfig = new Dictionary<string, EndpointSettings>
                    {
                        [networkId] = new EndpointSettings { EndpointID = networkId, Aliases = new List<string> { Name } }
                    }
                },
                Env = Env,
                Healthcheck = healthConfig,
                HostConfig = new HostConfig
                {
                    PortBindings = portBindings,
                    Binds = binds
                },
                Cmd = commands
            };
        }


        public void SetHealthCheck(List<string> testCommands)
        {
            healthConfig = new HealthConfig
            {
                Test = testCommands,
                Interval = new TimeSpan(0, 0, 0, 5),
                StartPeriod = 5 * 1000_0000_000,
                Retries = 50
            };
        }

        public void AddPortMapping(Protocol protocol, int containerPort, string hostAddress, int hostPort)
        {
            portBindingsConfig.Add(new PortMapping
            {
                Protocol = protocol,
                ContainerPort = containerPort,
                HostAddress = hostAddress,
                HostPort = hostPort
            });
        }

        public void AddVolumeMappings(string externalPath, string internalPath)
        {
            VolumeMappings[CorrectForGithubAction(externalPath)] = internalPath;
        }

        public void AddVolumeContentMappings(string content, string internalPath)
        {
            var directory = Path.GetFullPath(Path.Combine(Path.Combine(Path.Combine(Application.dataPath, "../Temp/"), Name),
                internalPath.Replace("/", "")));
            Directory.CreateDirectory(directory);
            var file = Path.Combine(directory, "file");
            File.WriteAllText(file, content);
            AddVolumeMappings(file, internalPath);
        }


        public void AddEnv(string name, string value)
        {
            Env.Add(name + "=" + value);
        }


        public void AddCommand(string command)
        {
            commands.Add(command);
        }

        /// <summary>
        /// Так как в github action используется docker in docker - то требуется коррекция пути
        /// </summary>
        /// <param name="externalPath"></param>
        /// <returns></returns>
        private static string CorrectForGithubAction(string externalPath)
        {
            return externalPath.Replace("/github/workspace/", "/home/runner/work/platform/platform/");
        }
    }


    public struct DockerImage
    {
        public string Name;
        private string Tag;
        private string Repo;
        public string Ref => (Repo != null ? Repo + "/" : "") + Name + ":" + Tag;

        public static DockerImage From(string repo, string image, string tag)
        {
            var dockerImage = new DockerImage
            {
                Name = image,
                Tag = tag,
                Repo = repo
            };
            return dockerImage;
        }

        public static DockerImage From(string reference)
        {
            var splitted = reference.Split(':');
            return From(null, splitted[0], splitted[1]);
        }
    }
}