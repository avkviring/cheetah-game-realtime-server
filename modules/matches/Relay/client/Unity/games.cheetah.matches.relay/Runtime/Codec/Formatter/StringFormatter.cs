using System;
using System.Runtime.CompilerServices;
using System.Text;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Formatter
{
    public sealed class StringFormatter : Formatter<string>, ArrayFormatter<string>
    {
        public static readonly StringFormatter Instance = new();
        private static readonly UnmanagedFormatter<byte> ByteFormatter = Formatter.ByteFormatter.Instance;

        public unsafe string Read(ref CheetahBuffer buffer)
        {
            var length = ByteFormatter.Read(ref buffer);
            var bytes = stackalloc byte[255];
            ByteFormatter.ReadFixedArray(ref buffer, bytes, length, 0);
            return Encoding.UTF8.GetString(bytes, length);
        }

        public unsafe void Write(string value, ref CheetahBuffer buffer)
        {
            if (value == null)
            {
                throw new NullValueNotSupportedInCodecException();
            }

            fixed (char* chars = value)
            {
                var bytes = stackalloc byte[255];
                var length = Encoding.UTF8.GetBytes(chars, value.Length, bytes, 255);
                if (length > 255)
                {
                    throw new OverflowException("Length string value should less 255 bytes");
                }

                ByteFormatter.Write((byte)length, ref buffer);
                ByteFormatter.WriteFixedArray(bytes, (uint)length, 0, ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref CheetahBuffer buffer, string[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(string[] value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }
    }

    public class NullValueNotSupportedInCodecException : Exception
    {
    }
}