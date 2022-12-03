namespace Cheetah.Matches.Realtime.UDS.FFI
{
    internal static class Const
    {
#if UNITY_64
#if UNITY_EDITOR_WIN || UNITY_STANDALONE_WIN
        public const string Library = "cheetah_matches_realtime_server_plugin";
#else
        public const string Library = "libcheetah_matches_realtime_server_plugin";
#endif
#else
        public const string Library = "cheetah_matches_realtime_server_plugin";
#endif
    }
}