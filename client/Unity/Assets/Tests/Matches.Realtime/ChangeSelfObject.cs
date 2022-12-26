using System.Threading;
using NUnit.Framework;
using Shared;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
{
    public class ChangeSelfObject : AbstractTest
    {
        [Test]
        public void ShouldSetFromNotOwner()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // ждем отправки команды
            Thread.Sleep(200);
            // изменяем на другом клиенте
            clientB.Writer.SetLong(in createdObject.ObjectId, ScoreFieldId, 155);
            Thread.Sleep(200);
            // прием команды
            clientA.Update();
            // проверяем результат
            var changes = clientA.Reader.GetModifiedLongs(777, ScoreFieldId);
            var actual = changes[createdObject.ObjectId];
            Assert.AreEqual(155, actual);
        }
    }
}