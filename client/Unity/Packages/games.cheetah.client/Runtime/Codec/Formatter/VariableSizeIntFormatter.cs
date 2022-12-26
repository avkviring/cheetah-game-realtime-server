using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class VariableSizeIntFormatter : Formatter<int>, FixedArrayFormatter<int>, ArrayFormatter<int>
    {
        public static readonly VariableSizeIntFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public int Read(ref NetworkBuffer buffer)
        {
            return (int)VariableSizeLongFormatter.StaticRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(int value, ref NetworkBuffer buffer)
        {
            VariableSizeLongFormatter.StaticWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void ReadFixedArray(ref NetworkBuffer buffer, int* value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void WriteFixedArray(int* value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref NetworkBuffer buffer, int[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(int[] value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }
    }
}