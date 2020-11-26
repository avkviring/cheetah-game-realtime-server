namespace CheetahRelay
{
    public static class Const
    {
#if UNITY_ANDROID
        //private const string Import = "__Internal";
        public const string Library = "cheetah_relay_client";
#else
        public const string Library = "libcheetah_relay_client";
#endif


        public const ushort MaxFieldsInObject = 255;
        public const ushort MaxSizeStruct = 255;
        public const ushort AllStructuresSize = MaxFieldsInObject * MaxSizeStruct;
    }
}