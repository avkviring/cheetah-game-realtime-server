namespace Cheetah.Matches.Realtime.Internal.FFI
{
    internal static class Const
    {
#if UNITY_ANDROID
        public const string Library = "android-amrv7";
#endif
#if UNITY_STANDALONE_WIN || UNITY_EDITOR_WIN
        public const string Library = "win";
#endif
#if UNITY_STANDALONE_LINUX || UNITY_EDITOR_LINUX
  public const string Library = "linux";
#endif
#if UNITY_STANDALONE_OSX
        public const string Library = "macos";
#endif
#if UNITY_IOS
        public const string Library = "__Internal";
#endif
        public const ushort MaxSizeStruct = 255;
        public const ushort MaxFields = 255;
    }
}