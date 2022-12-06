using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public class IntFormatter : UnmanagedFormatter<int>
    {
        public static readonly IntFormatter Instance = new();


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override int UncheckedRead(ref CheetahBuffer buffer)
        {
            return (int)UIntFormatter.StaticUncheckedRead(ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(int value, ref CheetahBuffer buffer)
        {
            UIntFormatter.StaticUncheckedWrite((uint)value, ref buffer);
        }
    }
}