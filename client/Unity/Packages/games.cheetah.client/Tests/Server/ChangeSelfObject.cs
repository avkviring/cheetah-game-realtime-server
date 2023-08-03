using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
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
            var actual = changes.SearchFirst(it=>it.Item1==createdObject.ObjectId).Item2;
            Assert.AreEqual(155, actual);
            changes.Dispose();
        }
    }
}