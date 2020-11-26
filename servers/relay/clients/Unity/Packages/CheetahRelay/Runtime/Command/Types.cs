using System.Runtime.InteropServices;
using System.Text;

namespace CheetahRelay
{
    [StructLayout(LayoutKind.Sequential)]
    public struct RelayObjectId
    {
        public uint id;
        public uint user;

        public override string ToString()
        {
            return "RelayObjectId (id=" + id + ", user=" + user + ")";
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct CommandMeta
    {
        public ulong timestamp;
        public uint sourceUser;

        public override string ToString()
        {
            return "CommandMeta (timestamp=" + timestamp + ", sourceUser=" + sourceUser + ")";
        }
    }


    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct RelayBuffer
    {
        public byte size;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxSizeStruct)]
        public fixed byte values[Const.MaxSizeStruct];

        public RelayBuffer Add(byte value)
        {
            values[size] = value;
            size++;
            return this;
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

        public void Clear()
        {
            size = 0;
        }
    }

    public enum LogLevel
    {
        Info,
        Warn,
        Error,
    }

    public enum ConnectionStatus
    {
        Connecting,
        Connected,
        Disconnected
    }
}