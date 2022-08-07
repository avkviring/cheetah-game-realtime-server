using System;
using System.Threading;
using System.Threading.Tasks;
using Docker.DotNet;

namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    /// <summary>
    /// Ожидаем готовности приложения, с учетом HealthCheck
    /// </summary>
    public static class WaitingForLaunch
    {
        public static async Task Wait(DockerClient docker, string containerId, string name)
        {
            while (true)
            {
                var inspect = await docker.Containers.InspectContainerAsync(containerId, CancellationToken.None);
                var status = inspect.State;
                if (status.Dead)
                {
                    throw new StartContainerException(name, "dead", status.Error);
                }

                if (status.Status == "exited")
                {
                    throw new StartContainerException(name, "exited", status.Error);
                }

                if (status.Running)
                {
                    if (status.Health != null)
                    {
                        var health = status.Health;
                        switch (health.Status)
                        {
                            case "none":
                                return;
                            case "healthy":
                                return;
                            case "unhealthy":
                                throw new StartContainerException(name, "unhealthy", status.Error);
                        }
                    }
                    else
                    {
                        return;
                    }
                }
            }
        }
    }

    public class StartContainerException : Exception
    {
        public StartContainerException(string containerId, string state, string message) : base(
            containerId + " has " + state +
            (message.Length > 0 ? " with message " + message : "")
        )
        {
        }
    }
}