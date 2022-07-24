using System.Runtime.CompilerServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Formatter
{
    public sealed class UShortFormatter : UnmanagedFormatter<ushort>
    {
        public static readonly UShortFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override ushort UncheckedRead(ref CheetahBuffer buffer)
        {
            return StaticUncheckedRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(ushort value, ref CheetahBuffer buffer)
        {
            StaticUncheckedWrite(value, ref buffer);
        }

        public static unsafe void StaticUncheckedWrite(ushort value, ref CheetahBuffer buffer)
        {
            buffer.values[buffer.size++] = (byte)((value & 0xFF00) >> 8);
            buffer.values[buffer.size++] = (byte)(value & 0xFF);
        }

        public static unsafe ushort StaticUncheckedRead(ref CheetahBuffer buffer)
        {
            return (ushort)((buffer.values[buffer.pos++] << 8) + buffer.values[buffer.pos++]);
        }
    }
}