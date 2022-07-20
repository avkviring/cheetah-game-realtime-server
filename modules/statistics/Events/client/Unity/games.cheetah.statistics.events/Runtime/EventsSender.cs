using System.Collections.Generic;
using UnityEngine.Assertions;

namespace Cheetah.Statistics.Events
{
    /// <summary>
    /// Сохранение на сервере произвольных событий
    /// </summary>
    public class EventsSender
    {
        private readonly StatisticsSession session;

        public EventsSender(StatisticsSession session)
        {
            Assert.IsNotNull(session, "session != null");
            this.session = session;
        }

        public void Send(string value, Dictionary<string, string> labels)
        {
            labels["type"] = "event";
            session.Send(value, labels);
        }

        public void Send(string value)
        {
            session.Send(value, new());
        }
    }
}