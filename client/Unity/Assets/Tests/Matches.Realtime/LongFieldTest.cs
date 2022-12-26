using System.Threading;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
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