using System.Runtime.InteropServices;
using System.Text;

namespace CheetahRelay.Runtime.LowLevel.External
{
    /**
     * Структура для передачи данных из/в Rust часть клиента
     */
    [StructLayout(LayoutKind.Sequential)]
    public struct Command
    {
        public const ushort BufferSize = 255;
        public S2CCommandType commandTypeS2C;
        public C2SCommandType commandTypeC2S;
        public ushort fieldId;
        public RelayGameObjectId objectId;
        public Bytes structureData;
        public Bytes eventData;
        public long longValue;
        public double floatValue;
        public ulong accessGroup;
        public LongCounters longCounters;
        public DoubleCounters doubleCounters;
        public Structures structures;
    }

    public enum C2SCommandType
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

    public enum S2CCommandType
    {
        Upload,
        SetLongCounter,
        SetFloatCounter,
        Structure,
        Event,
        Unload,
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct RelayGameObjectId
    {
        public uint id;
        public ushort client;
        public RelayGameObjectIdType type;
    }

    public enum RelayGameObjectIdType
    {
        Root,
        Current,
        Client,
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct Structures
    {
        public byte count;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed ushort fields[Command.BufferSize];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed byte sizes[Command.BufferSize];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize * Command.BufferSize)]
        public fixed byte values[Command.BufferSize * Command.BufferSize];
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct LongCounters
    {
        public byte count;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed ushort fields[Command.BufferSize];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed long values[Command.BufferSize];
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct DoubleCounters
    {
        public byte count;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed ushort fields[Command.BufferSize];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed double values[Command.BufferSize];
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct Bytes
    {
        public byte size;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Command.BufferSize)]
        public fixed byte values[Command.BufferSize];

        public void AddValue(byte value)
        {
            values[size] = value;
            size++;
        }

        public override string ToString()
        {
            var builder = new StringBuilder();
            builder.Append("Bytes[size = " + size + ", data=(");
            for (var i = 0; i < size; i++)
            {
                if (i > 0)
                {
                    builder.Append(" ");
                }
                builder.Append(values[i].ToString("X2"));
            }

            builder.Append(")]");

            return builder.ToString();
        }
    }

    public enum LogLevel
    {
        Info,
        Warn,
        Error,
    }

    public enum NetworkStatus
    {
        None,
        Connecting,
        OnLine,
        Disconnected,
    }
}