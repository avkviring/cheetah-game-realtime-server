using Cheetah.Matches.Realtime.Codec.Formatter;
using Cheetah.Matches.Realtime.Types;
using UnityEngine;

namespace Cheetah.Matches.Realtime.Codec.Standard
{
    public class Vector2Codec : Codec<Vector2>
    {
        public void Decode(ref CheetahBuffer buffer, ref Vector2 dest)
        {
            buffer.AssertEnoughData(sizeof(float) * 2);
            dest.x = FloatFormatter.StaticUncheckedRead(ref buffer);
            dest.y = FloatFormatter.StaticUncheckedRead(ref buffer);
        }

        public void Encode(in Vector2 source, ref CheetahBuffer buffer)
        {
            buffer.AssertFreeSpace(sizeof(float) * 2);
            FloatFormatter.StaticUncheckedWrite(source.x, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.y, ref buffer);
        }
    }
}