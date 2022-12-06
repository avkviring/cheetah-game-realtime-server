using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class VariableSizeUIntFormatter : Formatter<uint>, FixedArrayFormatter<uint>, ArrayFormatter<uint>
    {
        public static readonly VariableSizeUIntFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public uint Read(ref CheetahBuffer buffer)
        {
            return (uint)VariableSizeULongFormatter.StaticRead(ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(uint value, ref CheetahBuffer buffer)
        {
            VariableSizeULongFormatter.StaticWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void ReadFixedArray(ref CheetahBuffer buffer, uint* value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void WriteFixedArray(uint* value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref CheetahBuffer buffer, uint[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(uint[] value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }
    }
}