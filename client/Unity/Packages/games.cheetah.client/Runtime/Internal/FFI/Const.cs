namespace Games.Cheetah.Client.Internal.FFI
{
    internal static class Const
    {
#if UNITY_EDITOR_WIN
        public const string Library = "windows";
#elif UNITY_EDITOR_LINUX
        public const string Library = "linux";            
#elif UNITY_ANDROID
        public const string Library = "cheetah_client";
#elif UNITY_IOS
        public const string Library = "__Internal";
#elif UNITY_STANDALONE_WIN
        public const string Library = "windows";
#elif UNITY_STANDALONE_LINUX
        public const string Library = "linux";
#elif UNITY_STANDALONE_OSX
        public const string Library = "macos";
#endif

        public const ushort MaxSizeStruct = 255;
        public const ushort MaxFields = 255;
    }
}