using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Internal;
using NUnit.Framework;

namespace Tests.Matches.Pride
{
    public class ErrorTest
    {
        /**
         * Проверяем передачу ошибок из FFI
         */
        [Test]
        public void ShouldException()
        {
            var wrongHost = "____";
            Assert.Throws<CreateClientError>(() => new CheetahClient(wrongHost, 0, 0, 0, new byte[128], new CodecRegistryBuilder().Build()));
        }
    }
}