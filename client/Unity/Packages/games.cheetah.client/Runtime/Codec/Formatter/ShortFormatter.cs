using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class ShortFormatter : UnmanagedFormatter<short>
    {
        public static readonly ShortFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override short UncheckedRead(ref NetworkBuffer buffer)
        {
            return (short)UShortFormatter.StaticUncheckedRead(ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(short value, ref NetworkBuffer buffer)
        {
            UShortFormatter.StaticUncheckedWrite((ushort)value, ref buffer);
        }
    }
}