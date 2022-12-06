using System;
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
            var wrongHost = new Uri("udp://not-exist-host:8080");
            Assert.Throws<CreateClientError>(() => new CheetahClient(wrongHost, 0, 0, new byte[128], new CodecRegistryBuilder().Build()));
        }
    }
}