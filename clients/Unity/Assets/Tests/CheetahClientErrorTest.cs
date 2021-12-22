using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Internal;
using NUnit.Framework;

namespace Tests
{
    public class CheetahClientErrorTest
    {
        [Test]
        public void ShouldException()
        {
            Assert.Throws<CreateClientError>(() => new CheetahClient("____", 0, 0, 0, new byte[128], new CodecRegistry()));
        }
    }
}