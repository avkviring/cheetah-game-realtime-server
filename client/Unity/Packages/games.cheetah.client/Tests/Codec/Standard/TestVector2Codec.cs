using Games.Cheetah.Client.Codec.Standard;
using Games.Cheetah.Client.Types.Field;
using NUnit.Framework;
using UnityEngine;

namespace Games.Cheetah.Client.Tests.Codec.Standard
{
    public class TestVector2Codec
    {
        [Test]
        public void TestCodec()
        {
            var id = new Vector2
            {
                x = (float)0.1,
                y = (float)0.2,
            };
            var buffer = new NetworkBuffer();
            var codec = new Vector2Codec();
            codec.Encode(in id, ref buffer);

            var decoded = new Vector2();
            codec.Decode(ref buffer, ref decoded);
            Assert.AreEqual(id, decoded);
        }
    }
}