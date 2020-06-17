using System;
using System.Runtime.InteropServices;

namespace Cheetach.Relay
{
    [StructLayout(LayoutKind.Sequential)]
    public struct CommandFFI
    {
        public S2CCommandFFIType commandTypeS2C;
        public C2SCommandFFIType commandTypeC2S;
        public UInt16 fieldId;
        public ObjectId objectId;
        public FieldFFIBinary structureData;
        public FieldFFIBinary eventData;
        public Int64 longValue;
        public double floatValue;
        public UInt64 accessGroup;
        public long_counters: FieldsFFI<i64>;
        public float_counters: FieldsFFI<f64>;
        public structures: FieldsFFI<FieldFFIBinary>;
    }

    public enum C2SCommandFFIType
    {
        Upload,
        IncrementLongCounter,
        SetLongCounter,
        IncrementFloatCounter,
        SetFloatCounter,
        Structure,
        Event,
        Unload,
    }

    public enum S2CCommandFFIType
    {
        Upload,
        SetLongCounter,
        SetFloatCounter,
        Structure,
        Event,
        Unload,
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct ObjectId
    {
        public UInt32 id;
        public UInt16 client;
        public ObjectIdType idType;
    }

    public enum ObjectIdType
    {
        Root,
        Current,
        Client,
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FieldFFIBinary
    {
        public UInt64 binarySize;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 255)]
        public byte[] value;
    }
}