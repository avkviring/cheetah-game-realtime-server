using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using UnityEngine;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class Vector3Codec : Codec<Vector3>
    {
        public void Decode(ref CheetahBuffer buffer, ref Vector3 dest)
        {
            buffer.AssertEnoughData(sizeof(float) * 3);
            dest.x = FloatFormatter.StaticUncheckedRead(ref buffer);
            dest.y = FloatFormatter.StaticUncheckedRead(ref buffer);
            dest.z = FloatFormatter.StaticUncheckedRead(ref buffer);
        }

        public void Encode(in Vector3 source, ref CheetahBuffer buffer)
        {
            buffer.AssertFreeSpace(sizeof(float) * 3);
            FloatFormatter.StaticUncheckedWrite(source.x, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.y, ref buffer);
            FloatFormatter.StaticUncheckedWrite(source.z, ref buffer);
        }
    }
}