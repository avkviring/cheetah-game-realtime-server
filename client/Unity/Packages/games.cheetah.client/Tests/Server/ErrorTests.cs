using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class ErrorTest
    {
        /**
         * Проверяем передачу ошибок из FFI
         */
        [Test]
        public void ShouldException()
        {
            Assert.Throws<CreateClientError>(() => new NetworkClient(0, "_wrongHost", 5555, 0, 0, new byte[128], new CodecRegistryBuilder().Build()));
        }
    }
}