using Cheetah.Matches.Relay.Codec.Standard;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;
using UnityEngine;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Standard
{
    public class TestVector3Codec
    {
        [Test]
        public void TestCodec()
        {
            var id = new Vector3
            {
                x = (float)0.1,
                y = (float)0.2,
                z = (float)0.3,
            };
            var buffer = new CheetahBuffer();
            var codec = new Vector3Codec();
            codec.Encode(ref id, ref buffer);

            var decoded = new Vector3();
            codec.Decode(ref buffer, ref decoded);
            Assert.AreEqual(id, decoded);
        }
    }
}