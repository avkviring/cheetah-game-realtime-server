using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class BoolFormatter : UnmanagedFormatter<bool>
    {
        public static readonly BoolFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override unsafe bool UncheckedRead(ref NetworkBuffer buffer)
        {
            return buffer.values[buffer.pos++] != 0;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override unsafe void UncheckedWrite(bool value, ref NetworkBuffer buffer)
        {
            buffer.values[buffer.size++] = (byte)(value ? 1 : 0);
        }
    }
}