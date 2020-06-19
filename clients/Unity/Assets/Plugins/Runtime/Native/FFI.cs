using System.Runtime.InteropServices;

namespace Cheetach.Relay
{
    /**
     * Структура для передачи данных из/в Rust часть клиента
     */
    [StructLayout(LayoutKind.Sequential)]
    public struct CommandFFI
    {
        public S2CCommandFFIType commandTypeS2C;
        public C2SCommandFFIType commandTypeC2S;
        public ushort fieldId;
        public ObjectId objectId;
        public FieldFFIBinary structureData;
        public FieldFFIBinary eventData;
        public long longValue;
        public double floatValue;
        public ulong accessGroup;
        public FieldsFFI<long> long_counters;
        public FieldsFFI<double> float_counters;
        public FieldsFFI<FieldFFIBinary> structures;
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
        public uint id;
        public ushort client;
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
        public byte binarySize;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 255)]
        public byte[] value;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FieldsFFI<T>
    {
        public byte size;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 255)]
        public FieldFFI<T>[] values;
    }

    public struct FieldFFI<T>
    {
        public ushort fieldId;
        public T value;
    }

    public enum LogLevel
    {
        Info,
        Warn,
        Error,
    }

    public enum NetworkStatus
    {
        Connecting,
        OnLine,
        Disconnected,
    }
}