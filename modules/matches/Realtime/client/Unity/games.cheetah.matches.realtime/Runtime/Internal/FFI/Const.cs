namespace Cheetah.Matches.Realtime.Internal.FFI
{
    internal static class Const
    {
#if UNITY_ANDROID || UNITY_STANDALONE_WIN || UNITY_EDITOR_WIN
        public const string Library = "cheetah_matches_relay_client";
#endif
#if UNITY_STANDALONE_LINUX || UNITY_EDITOR_LINUX
  public const string Library = "libcheetah_matches_relay_client";
#endif
#if UNITY_STANDALONE_OSX
        public const string Library = "libcheetah_matches_relay_client";
#endif
#if UNITY_IOS
        public const string Library = "__Internal";
#endif
        public const ushort MaxSizeStruct = 255;
        public const ushort MaxFields = 255;
    }
}