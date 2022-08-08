namespace Cheetah.Matches.Realtime.Internal.FFI
{
    internal static class Const
    {
#if UNITY_EDITOR_WIN
        public const string Library = "win";
#elif UNITY_EDITOR_LINUX
        public const string Library = "linux";            
#elif UNITY_ANDROID
        public const string Library = "android-armv7";
#elif UNITY_IOS
        public const string Library = "__Internal";
#elif UNITY_STANDALONE_WIN
        public const string Library = "win";
#elif UNITY_STANDALONE_LINUX
        public const string Library = "linux";
#elif UNITY_STANDALONE_OSX
        public const string Library = "macos";
#endif

        public const ushort MaxSizeStruct = 255;
        public const ushort MaxFields = 255;
    }
}