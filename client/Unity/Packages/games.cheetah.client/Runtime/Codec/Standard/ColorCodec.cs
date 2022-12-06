using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using UnityEngine;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class ColorCodec : Codec<Color>
    {
        public void Decode(ref CheetahBuffer buffer, ref Color dest)
        {
            buffer.AssertEnoughData(sizeof(float) * 4);
            var r = FloatFormatter.StaticUncheckedRead(ref buffer);
            var g = FloatFormatter.StaticUncheckedRead(ref buffer);
            var b = FloatFormatter.StaticUncheckedRead(ref buffer);
            var a = FloatFormatter.StaticUncheckedRead(ref buffer);
            dest = new Color(r, g, b, a);
        }

        public void Encode(in Color source, ref CheetahBuffer buffer)
        {
            buffer.AssertFreeSpace(sizeof(float) * 4);
            FloatFormatter.StaticUncheckedWrite(source.r, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.g, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.b, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.a, ref buffer);
        }
    }
}