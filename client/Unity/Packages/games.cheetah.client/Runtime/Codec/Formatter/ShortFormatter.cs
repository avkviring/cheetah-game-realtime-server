using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class ShortFormatter : UnmanagedFormatter<short>
    {
        public static readonly ShortFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override short UncheckedRead(ref CheetahBuffer buffer)
        {
            return (short)UShortFormatter.StaticUncheckedRead(ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(short value, ref CheetahBuffer buffer)
        {
            UShortFormatter.StaticUncheckedWrite((ushort)value, ref buffer);
        }
    }
}