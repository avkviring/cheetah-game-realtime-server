using Cheetah.Matches.Relay.Codec.Standard;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;
using UnityEngine;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Standard
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
            var buffer = new CheetahBuffer();
            var codec = new Vector2Codec();
            codec.Encode(ref id, ref buffer);

            var decoded = new Vector2();
            codec.Decode(ref buffer, ref decoded);
            Assert.AreEqual(id, decoded);
        }
    }
}