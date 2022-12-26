using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using UnityEngine;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class Vector2Codec : Codec<Vector2>
    {
        public void Decode(ref NetworkBuffer buffer, ref Vector2 dest)
        {
            buffer.AssertEnoughData(sizeof(float) * 2);
            dest.x = FloatFormatter.StaticUncheckedRead(ref buffer);
            dest.y = FloatFormatter.StaticUncheckedRead(ref buffer);
        }

        public void Encode(in Vector2 source, ref NetworkBuffer buffer)
        {
            buffer.AssertFreeSpace(sizeof(float) * 2);
            FloatFormatter.StaticUncheckedWrite(source.x, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.y, ref buffer);
        }
    }
}