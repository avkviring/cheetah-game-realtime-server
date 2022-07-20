using System;

namespace Cheetah.Platform.Editor.LocalServer.CheetahRegistry
{
    public class CheetahRegistryAuthException : Exception
    {
        public CheetahRegistryAuthException() : base("CheetahDockerRegistryAuthException")
        {
        }
    }
}