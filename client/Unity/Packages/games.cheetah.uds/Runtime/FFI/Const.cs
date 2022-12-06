namespace Games.Cheetah.UDS.FFI
{
    internal static class Const
    {
#if UNITY_64
#if UNITY_EDITOR_WIN || UNITY_STANDALONE_WIN
        public const string Library = "cheetah_plugin";
#else
        public const string Library = "libcheetah_plugin";
#endif
#else
        public const string Library = "cheetah_plugin";
#endif
    }
}