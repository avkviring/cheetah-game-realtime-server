using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class LongFieldTest : AbstractTest
    {
        [Test]
        public void ShouldSet()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            clientA.Writer.SetLong(in createdObject.ObjectId, ScoreFieldId, 155);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var changes = clientB.Reader.GetModifiedLongs(777, ScoreFieldId);
            var actual = changes[createdObject.ObjectId];
            Assert.AreEqual(155, actual);
        }


        [Test]
        public void ShouldIncrement()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.Increment(in createdObject.ObjectId, ScoreFieldId, 1001L);
            clientA.Writer.Increment(in createdObject.ObjectId, ScoreFieldId, 2002L);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var changes = clientB.Reader.GetModifiedLongs (777, ScoreFieldId);
            var actual = changes[createdObject.ObjectId];
            Assert.AreEqual(3003, actual);
        }
    }
}