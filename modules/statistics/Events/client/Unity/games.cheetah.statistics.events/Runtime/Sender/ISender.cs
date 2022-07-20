using System.Collections.Generic;

namespace Cheetah.Statistics.Events.Sender
{
    /// <summary>
    /// Интерфейс для отправки сообщений в внешнию систему
    /// </summary>
    public interface ISender
    {
        void Send(string value, Dictionary<string, string> labels);
    }
}