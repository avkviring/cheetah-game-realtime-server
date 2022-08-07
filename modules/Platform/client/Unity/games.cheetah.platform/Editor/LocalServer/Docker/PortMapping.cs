namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    public struct PortMapping
    {
        public Protocol Protocol;
        public int ContainerPort;
        public int HostPort;
        public string HostAddress;
    }
}