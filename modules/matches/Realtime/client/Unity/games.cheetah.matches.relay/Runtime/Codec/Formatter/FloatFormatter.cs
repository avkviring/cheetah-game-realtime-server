using System.Runtime.CompilerServices;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Formatter
{
    public sealed class FloatFormatter : UnmanagedFormatter<float>
    {
        public static readonly FloatFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override float UncheckedRead(ref CheetahBuffer buffer)
        {
            return StaticUncheckedRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(float value, ref CheetahBuffer buffer)
        {
            StaticUncheckedWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe void StaticUncheckedWrite(float value, ref CheetahBuffer buffer)
        {
            UIntFormatter.StaticUncheckedWrite(*(uint*)&value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe float StaticUncheckedRead(ref CheetahBuffer buffer)
        {
            var read = UIntFormatter.StaticUncheckedRead(ref buffer);
            return *(float*)&read;
        }
    }
}