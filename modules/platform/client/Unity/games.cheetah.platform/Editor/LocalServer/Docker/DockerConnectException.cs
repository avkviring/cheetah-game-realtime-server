using System;

namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    public class DockerConnectException : Exception
    {
        public DockerConnectException(Exception e) : base(e.Message, e)
        {
        }
    }
}