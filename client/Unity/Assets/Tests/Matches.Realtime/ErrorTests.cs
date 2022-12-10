using Games.Cheetah.Client;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using NUnit.Framework;

namespace Tests.Matches.Realtime
{
    public class ErrorTest
    {
        /**
         * Проверяем передачу ошибок из FFI
         */
        [Test]
        public void ShouldException()
        {
            Assert.Throws<CreateClientError>(() => new CheetahClient("_wrongHost", 5555, 0, 0, new byte[128], new CodecRegistryBuilder().Build()));
        }
    }
}