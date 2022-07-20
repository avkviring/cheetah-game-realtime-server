using System;
using System.Text;
using Cheetah.Platform.Editor.LocalServer.Runner;
using Docker.DotNet.Models;

namespace Cheetah.Platform.Editor.LocalServer.Docker
{
    /// <summary>
    ///     Отображение прогресса получения docker image
    /// </summary>
    internal class ImageCreateProgress : IProgress<JSONMessage>
    {
        private readonly IDockerProgressListener progressListener;
        private readonly string title;
        private int layers;

        public ImageCreateProgress(IDockerProgressListener progressListener, string title)
        {
            this.progressListener = progressListener;
            this.title = title;
        }

        public void Report(JSONMessage value)
        {
            switch (value.Status)
            {
                case "Pulling fs layer":
                    layers++;
                    return;
                case "Download complete":
                    layers--;
                    return;
                case "Verifying Checksum":
                case "Pull complete":
                    return;
            }


            var progressInfo = new StringBuilder();
            if (value.Progress != null)
            {
                var remaining = FormatValue(value.Progress.Total - value.Progress.Current);
                progressInfo.Append(value.Status + " " + title + ":");
                progressInfo.Append(" layer " + layers + " - remaining " + remaining);
                progressListener.SetProgressTitle(progressInfo.ToString());
            }
        }

        private string FormatValue(long value)
        {
            if (value > 1024 * 1024) return value / (1024 * 1024) + " MB";

            if (value > 1024) return value / (1024 * 1024) + " KB";

            return value + " B";
        }
    }
}