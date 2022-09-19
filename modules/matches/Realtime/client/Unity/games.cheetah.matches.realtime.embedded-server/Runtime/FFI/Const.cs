namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Const
    {
#if UNITY_EDITOR_WIN || UNITY_STANDALONE_WIN
        public const string Library = "cheetah_matches_realtime_embedded";
#elif UNITY_EDITOR_LINUX || UNITY_STANDALONE_LINUX
        public const string Library = "libcheetah_matches_realtime_embedded";
#elif UNITY_EDITOR_OSX || UNITY_STANDALONE_OSX
        public const string Library = "libcheetah_matches_realtime_embedded";
#else
    public const string Library = "not-supported";
#endif
    }
}