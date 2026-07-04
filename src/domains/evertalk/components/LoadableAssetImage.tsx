import { useFirstLoadableImage } from '../hooks';
import type { LoadableAssetImageProps } from '../types';

export function LoadableAssetImage({ candidates, alt, className, fallback }: LoadableAssetImageProps) {
  const [src, useNextImage] = useFirstLoadableImage(candidates);

  if (!src) {
    return <>{fallback}</>;
  }

  return <img className={className} src={src} alt={alt} onError={useNextImage} />;
}
