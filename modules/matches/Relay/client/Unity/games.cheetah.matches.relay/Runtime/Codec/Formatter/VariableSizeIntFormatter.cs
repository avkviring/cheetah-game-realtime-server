using System.Runtime.CompilerServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Formatter
{
    public sealed class VariableSizeIntFormatter : Formatter<int>, FixedArrayFormatter<int>, ArrayFormatter<int>
    {
        public static readonly VariableSizeIntFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public int Read(ref CheetahBuffer buffer)
        {
            return (int)VariableSizeLongFormatter.StaticRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(int value, ref CheetahBuffer buffer)
        {
            VariableSizeLongFormatter.StaticWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void ReadFixedArray(ref CheetahBuffer buffer, int* value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void WriteFixedArray(int* value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref CheetahBuffer buffer, int[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(int[] value, uint size, uint offset, ref CheetahBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }
    }
}