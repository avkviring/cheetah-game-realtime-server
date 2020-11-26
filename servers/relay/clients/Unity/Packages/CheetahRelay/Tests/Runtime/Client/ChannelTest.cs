using NUnit.Framework;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class ChannelTest : AbstractCommandTest
    {
        /**
         * Проверяем только процесс установки канала, тестирование собственно переключение делаем в rust
         */
        [Test]
        public void Test()
        {
            ClientCommands.SetCurrentClient(clientA);
            Assert.True(ChannelCommands.SetChannel(Channel.UnreliableUnordered, 0));
        }
    }
}