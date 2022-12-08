namespace Games.Cheetah.Client.Internal.FFI
{
    internal static class Const
    {
#if UNITY_IOS
        public const string Library = "__Internal";
#else
        public const string Library = "cheetah_client";
#endif
        public const ushort MaxSizeStruct = 255;
        public const ushort MaxFields = 255;
    }
}