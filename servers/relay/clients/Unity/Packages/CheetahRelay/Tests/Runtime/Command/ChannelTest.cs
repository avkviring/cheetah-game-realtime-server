using NUnit.Framework;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class ChannelTest : AbstractTest
    {
        /**
         * Проверяем только процесс установки канала, тестирование собственно переключение делаем в rust
         */
        [Test]
        public void Test()
        {
            CheetahClient.SetCurrentClient(clientA);
            Assert.True(CheetahClient.SetChannelType(CheetahClient.ChannelType.UnreliableUnordered, 0));
        }
    }
}