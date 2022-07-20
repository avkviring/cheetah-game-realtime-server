using System.Runtime.CompilerServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Formatter
{
    public sealed class UIntFormatter : UnmanagedFormatter<uint>
    {
        public static readonly UIntFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override uint UncheckedRead(ref CheetahBuffer buffer)
        {
            return StaticUncheckedRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(uint value, ref CheetahBuffer buffer)
        {
            StaticUncheckedWrite(value, ref buffer);
        }

        public static unsafe uint StaticUncheckedRead(ref CheetahBuffer buffer)
        {
            return ((uint)buffer.values[buffer.pos++] << 24) +
                   ((uint)buffer.values[buffer.pos++] << 16) +
                   ((uint)buffer.values[buffer.pos++] << 8) + buffer.values[buffer.pos++];
        }

        public static unsafe void StaticUncheckedWrite(uint value, ref CheetahBuffer buffer)
        {
            buffer.values[buffer.size++] = (byte)((value & 0xFF000000) >> 24);
            buffer.values[buffer.size++] = (byte)((value & 0xFF0000) >> 16);
            buffer.values[buffer.size++] = (byte)((value & 0xFF00) >> 8);
            buffer.values[buffer.size++] = (byte)(value & 0xFF);
        }
    }
}