using System.Runtime.CompilerServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Formatter
{
    public sealed class BoolFormatter : UnmanagedFormatter<bool>
    {
        public static readonly BoolFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override unsafe bool UncheckedRead(ref CheetahBuffer buffer)
        {
            return buffer.values[buffer.pos++] != 0;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override unsafe void UncheckedWrite(bool value, ref CheetahBuffer buffer)
        {
            buffer.values[buffer.size++] = (byte)(value ? 1 : 0);
        }
    }
}