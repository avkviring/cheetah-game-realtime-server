namespace CheetahRelay
{
    public static class Const
    {
        
#if UNITY_ANDROID || UNITY_STANDALONE_LINUX
        public const string Library = "cheetah_relay_client";
#else
        public const string Library = "libcheetah_relay_client";
#endif

        
        public const ushort MaxSizeStruct = 255;
    }
}