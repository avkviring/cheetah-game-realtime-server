using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class VariableSizeLongFormatter : Formatter<long>, FixedArrayFormatter<long>, ArrayFormatter<long>
    {
        public static readonly VariableSizeLongFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public long Read(ref NetworkBuffer buffer)
        {
            return StaticRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void Write(long value, ref NetworkBuffer buffer)
        {
            StaticWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void ReadFixedArray(ref NetworkBuffer buffer, long* value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public unsafe void WriteFixedArray(long* value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ReadArray(ref NetworkBuffer buffer, long[] value, uint size, uint offset)
        {
            for (var i = 0; i < size; i++)
            {
                value[i + offset] = Read(ref buffer);
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void WriteArray(long[] value, uint size, uint offset, ref NetworkBuffer buffer)
        {
            for (var i = 0; i < size; i++)
            {
                Write(value[i + offset], ref buffer);
            }
        }

        public static long StaticRead(ref NetworkBuffer buffer)
        {
            var unsigned = VariableSizeULongFormatter.StaticRead(ref buffer);
            var half = (long)(unsigned >> 1);
            var value = unsigned % 2 == 0 ? half : ~half;
            return value;
        }

        public static void StaticWrite(long value, ref NetworkBuffer buffer)
        {
            var unsignedValue = (ulong)value;
            var zigzag = value < 0 ? ~unsignedValue * 2 + 1 : unsignedValue << 1;
            VariableSizeULongFormatter.StaticWrite(zigzag, ref buffer);
        }
    }
}