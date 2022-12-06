using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class ULongFormatter : UnmanagedFormatter<ulong>
    {
        public static readonly ULongFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override ulong UncheckedRead(ref CheetahBuffer buffer)
        {
            return StaticUncheckedRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(ulong value, ref CheetahBuffer buffer)
        {
            StaticUncheckedWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe void StaticUncheckedWrite(ulong value, ref CheetahBuffer buffer)
        {
            buffer.values[buffer.size++] = (byte)(value >> 56);
            buffer.values[buffer.size++] = (byte)((value & 0xFF000000000000) >> 48);
            buffer.values[buffer.size++] = (byte)((value & 0xFF0000000000) >> 40);
            buffer.values[buffer.size++] = (byte)((value & 0xFF00000000) >> 32);
            buffer.values[buffer.size++] = (byte)((value & 0xFF000000) >> 24);
            buffer.values[buffer.size++] = (byte)((value & 0xFF0000) >> 16);
            buffer.values[buffer.size++] = (byte)((value & 0xFF00) >> 8);
            buffer.values[buffer.size++] = (byte)(value & 0xFF);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe ulong StaticUncheckedRead(ref CheetahBuffer buffer)
        {
            return ((ulong)buffer.values[buffer.pos++] << 56) + ((ulong)buffer.values[buffer.pos++] << 48) +
                   ((ulong)buffer.values[buffer.pos++] << 40) + ((ulong)buffer.values[buffer.pos++] << 32) +
                   ((ulong)buffer.values[buffer.pos++] << 24) +
                   ((ulong)buffer.values[buffer.pos++] << 16) +
                   ((ulong)buffer.values[buffer.pos++] << 8) + buffer.values[buffer.pos++];
        }
    }
}