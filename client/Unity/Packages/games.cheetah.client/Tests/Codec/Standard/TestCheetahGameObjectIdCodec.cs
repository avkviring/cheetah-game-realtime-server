using Games.Cheetah.Client.Codec.Standard;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Standard
{
    public class TestCheetahGameObjectIdCodec
    {
        [Test]
        [TestCase(true, (ushort)0)]
        [TestCase(false, (ushort)125)]
        public void TestCodec(bool roomOwner, ushort memberId)
        {
            var id = new NetworkObjectId
            {
                id = 100,
                IsRoomOwner = roomOwner,
                memberId = memberId
            };
            var buffer = new NetworkBuffer();
            var codec = new CheetahObjectIdCodec();
            codec.Encode(in id, ref buffer);

            var decoded = new NetworkObjectId();
            codec.Decode(ref buffer, ref decoded);
            Assert.AreEqual(id, decoded);
        }
    }
}