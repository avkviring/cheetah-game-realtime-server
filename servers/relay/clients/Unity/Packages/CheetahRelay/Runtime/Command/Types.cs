using System.Runtime.InteropServices;
using System.Text;

namespace CheetahRelay
{
    [StructLayout(LayoutKind.Sequential)]
    public struct CheetahObjectId
    {
        public uint id;
        /// <summary>
        /// Владельцем объекта может быть или комната или пользователь
        /// </summary>
        public bool roomOwner;
        /// <summary>
        /// Публичный ключ пользователя - владельца объекта, применимо  если roomOwner = false
        /// </summary>
        public uint user_public_key;

        public override string ToString()
        {
            return "RelayObjectId (id=" + id + ", user=" + user_public_key + ")";
        }

        
        public override bool Equals(object obj) {
            return obj is CheetahObjectId other && Equals(other);
        }

        public bool Equals(CheetahObjectId other)
        {
            return id == other.id && roomOwner == other.roomOwner && user_public_key == other.user_public_key;
        }

        public override int GetHashCode()
        {
            unchecked
            {
                var hashCode = (int) id;
                hashCode = (hashCode * 397) ^ roomOwner.GetHashCode();
                hashCode = (hashCode * 397) ^ (int) user_public_key;
                return hashCode;
            }
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct CheetahCommandMeta
    {
        public ulong timestamp;
        public uint sourceUser;

        public override string ToString()
        {
            return "CommandMeta (timestamp=" + timestamp + ", sourceUser=" + sourceUser + ")";
        }
    }


    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct CheetahBuffer
    {
        public byte size;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxSizeStruct)]
        public fixed byte values[Const.MaxSizeStruct];

        public CheetahBuffer Add(byte value)
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

    public enum CheetahLogLevel
    {
        Info,
        Warn,
        Error,
    }

    public enum CheetahConnectionStatus
    {
        Connecting,
        Connected,
        Disconnected
    }
}