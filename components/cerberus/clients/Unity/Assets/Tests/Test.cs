using Games.Cheetah.Cerberus.Internal;
using Grpc.Core;
using NUnit.Framework;

namespace games.cheetah.unity.cerberus.Tests
{
    public class Test
    {
        /// <summary>
        /// Проверка на работоспособность связки клиент-сервер
        /// требуется запущенный сервер с redis
        /// </summary>
        [Test]
        public async void CheckService()
        {
            var channel = new Channel("127.0.0.1:5001", ChannelCredentials.Insecure);
            var client = new Cerberus.CerberusClient(channel);
            var request = new CreateTokenRequest();
            request.DeviceId = "device-id";
            request.UserId = "user-id";
            var result = await client.createAsync(request);
            Assert.True(result.Session.Length > 20);
        }
    }
}