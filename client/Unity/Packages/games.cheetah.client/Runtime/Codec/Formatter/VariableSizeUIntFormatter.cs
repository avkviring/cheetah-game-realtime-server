using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class VariableSizeUIntFormatter : Formatter<uint>, FixedArrayFormatter<uint>, ArrayFormatter<uint>
    {
        public static readonly VariableSizeUIntFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public uint Read(ref NetworkBuffer buffer)
        {
            return (uint)VariableSizeULongFormatter.StaticRead(ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(uint value, ref NetworkBuffer buffer)
        {
            VariableSizeULongFormatter.StaticWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void ReadFixedArray(ref NetworkBuffer buffer, uint* value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void WriteFixedArray(uint* value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref NetworkBuffer buffer, uint[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(uint[] value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }
    }
}