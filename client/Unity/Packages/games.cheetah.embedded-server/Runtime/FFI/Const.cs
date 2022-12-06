namespace Games.Cheetah.EmbeddedServer.FFI
{
    internal static class Const
    {
#if UNITY_64
#if UNITY_EDITOR_WIN || UNITY_STANDALONE_WIN
        public const string Library = "cheetah_embedded";
#else
    public const string Library = "libcheetah_embedded";
#endif
#else
        public const string Library = "cheetah_embedded";
#endif
    }
}